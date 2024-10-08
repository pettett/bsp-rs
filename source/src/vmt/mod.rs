// // Valve Material Type

// import { arrayRemove, assertExists } from "../util.js";
// import { SourceFileSystem } from "./Main.js";
// import { Color } from "../Color.js";

// export type VKFParamMap = { [k: string]: string };
// export type VKFPairUnit = string | number | VKFPair[];
// export type VKFPair<T extends VKFPairUnit = VKFPairUnit> = [string, T];

use std::{
    collections::HashMap,
    io::{self, Read, Seek},
    sync::{Arc, OnceLock},
};

use thiserror::Error;

use crate::binaries::BinaryData;

#[derive(Debug)]
pub enum VMTUnit {
    String(String),
    Number(f32),
    Map(HashMap<String, VMTUnit>),
}

#[derive(Debug, Default, Clone)]
pub struct VMT {
    pub source: String, //DEBUG
    pub shader: String,
    pub data: HashMap<String, String>,
    pub patch: OnceLock<Option<Arc<VMT>>>,
}

#[derive(Error, Debug)]
pub enum VMTError{
	#[error("Failed to load VMT `{0}`")]
	Invalid(String),
}

impl VMT {
    pub fn new(source: String, shader: String) -> Self {
        Self {
            source,
            shader: shader.to_ascii_lowercase(),
            data: Default::default(),
            patch: Default::default(),
        }
    }

    pub fn from_string(source: String) -> Result<Self, VMTError> {
        let mut s = source.as_str();
        consume_vmt(&mut s).map_err(|e|VMTError::Invalid(source))
    }

    pub fn shader(&self) -> &str {
        self.shader.as_str()
    }

    pub fn get_basetex(&self) -> Option<&str> {
        self.get("$basetexture")
    }
    pub fn get_basetex2(&self) -> Option<&str> {
        self.get("$basetexture2")
    }
    pub fn get_envmap(&self) -> Option<&str> {
        self.get("$envmap")
    }
    pub fn get(&self, param: &str) -> Option<&str> {
        if let Some(data) = self.data.get(param) {
            Some(data.as_str())
        } else if let Some(Some(patch)) = self.patch.get() {
            patch.get(param)
        } else {
            None
        }
    }
}

fn remove_comments(data: &mut String) {
    loop {
        if let Some(next_comment) = data.find(r"//") {
            if let Some(next_newline) = data[next_comment..].find("\n") {
                // replace up to next line
                data.replace_range(next_comment..next_comment + next_newline, "");
            } else {
                // replace to end of file
                data.replace_range(next_comment.., "");
            }
        } else {
            break;
        }
    }
}

fn view_up_to_line(data: &str) -> &str {
    if let Some(next_newline) = data[..].find("\n") {
        // View up to nextline character
        &data[..next_newline]
    } else {
        // View up to end of file
        data
    }
}

fn consume_line(data: &mut &str) -> Result<(), VMTError> {
    if let Some(next_newline) = data[..].find("\n") {
        // clip up to next new line
        *data = &data[next_newline + 1..];
        Ok(())
    } else {
        Err(VMTError::Invalid("".to_owned()))
    }
}

fn consume_string(data: &mut &str) -> Result<String, VMTError> {
    *data = data.trim();
    let str = if let Some(after) = data.find(char::is_whitespace) {
        let str = data[..after].trim().trim_matches('"').to_ascii_lowercase();

        *data = &data[after + 1..];

        str
    } else {
        data.trim().trim_matches('"').to_ascii_lowercase()
    };

    return Ok(str);
}

fn consume_word(data: &mut &str) -> Result<String, VMTError> {
    let next = if data.find('"').ok_or(VMTError::Invalid("".to_owned()))? == 0 { 1 } else { 0 };

    let after = data[next..].find(&['"', ' ', '\n']).ok_or(VMTError::Invalid("".to_owned()))?;

    let str = data[next..next + after]
        .trim()
        .trim_matches('"')
        .to_ascii_lowercase();

    *data = &data[next + after + 1..];

    return Ok(str);
}

fn consume_vmt(data: &mut &str) -> Result<VMT, VMTError> {
    *data = data.trim();

    let mut vmt = VMT::new(data.to_owned(), consume_word(data)?);

    loop {
        let mut line_data = view_up_to_line(data);

        let s1 = consume_string(&mut line_data);
        //TODO: Read things like ints/floats/vectors/sub maps
        let s2 = consume_string(&mut line_data);

        match (s1, s2) {
            (Ok(s1), Ok(s2)) => {
                vmt.data.insert(s1, s2);
            }
            _ => {}
        }

        // otherwise, consume lines until we find something, or break
        if consume_line(data).is_err() {
            break;
        }
    }
    Ok(vmt)
}

impl BinaryData for VMT {
    fn read<R: Read + Seek>(
        buffer: &mut std::io::BufReader<R>,
        max_size: Option<usize>,
    ) -> std::io::Result<Self> {
        let mut bytes = vec![0; max_size.unwrap()];
        buffer.read_exact(&mut bytes)?;

        let mut data = String::from_utf8(bytes).unwrap();
 
        remove_comments(&mut data);

        consume_vmt(&mut data.as_str())
            .map_err(|_| io::Error::new(io::ErrorKind::InvalidData, "Cannot read VMT"))
    }
}

#[cfg(test)]
mod vmt_tests {
    use crate::bsp::consts::LumpType;
    use crate::bsp::header::BSPHeader;
    use crate::vmt::{consume_vmt, remove_comments};
    use crate::vpk::VPKDirectory;
    use std::collections::HashSet;
    use std::path::{Path, PathBuf};

    const PATH: &str = "D:\\Program Files (x86)\\Steam\\steamapps\\common\\Portal 2\\portal2\\maps\\sp_a2_laser_intro.bsp";

    #[test]
    fn test_decode() {
        let mut data = r#"
"VertexLitGeneric"
{
     "$basetexture" "Models/Combine_soldier/Combine_elite"
     "$bumpmap" "models/combine_soldier/combine_elite_normal"
     // Hello "gamers"
	 "$envmap" "env_cubemap"
     "$normalmapalphaenvmapmask" 1
     "$envmapcontrast" 1
     "$model" 1
     "$selfillum" 1
}
		"#
        .to_owned();

        remove_comments(&mut data);
        println!("{}", data);

        let vmt = consume_vmt(&mut data.as_str()).unwrap();

        println!("{}", vmt.shader);
        println!("{:?}", vmt.data);
    }

    #[test]
    fn test_misc_dir() {
        let dir = VPKDirectory::load(Default::default(),PathBuf::from(
            "D:\\Program Files (x86)\\Steam\\steamapps\\common\\Half-Life 2\\hl2\\hl2_misc_dir.vpk",
        ))
        .unwrap();

        let mut shaders = HashSet::new();

        for (_d, files) in &dir.files["vmt"] {
            for (_file, data) in files {
                let Ok(vmt) = data.load_vmt(&dir) else {
                    continue;
                };
                shaders.insert(vmt.shader.to_ascii_lowercase());
            }
        }
        println!("{:?}", shaders);
    }

    #[test]
    fn test_misc_dir_p2() {
        let dir = VPKDirectory::load(Default::default(),PathBuf::from(
            "D:\\Program Files (x86)\\Steam\\steamapps\\common\\Portal 2\\portal2\\pak01_dir.vpk",
        ))
        .unwrap();

        let mut shaders = HashSet::new();

        for (_d, files) in &dir.files["vmt"] {
            for (_file, data) in files {
                let Ok(vmt) = data.load_vmt(&dir) else {
                    continue;
                };
                shaders.insert(vmt.shader.to_ascii_lowercase());
            }
        }
        println!("{:?}", shaders);
    }

    #[cfg(target_arch = "x86_64")]
    #[test]
    fn test_maps() {
        let (header, mut buffer) = BSPHeader::load(Path::new(PATH)).unwrap();

        let pak_header = header.get_lump_header(LumpType::PakFile);

        let dir: VPKDirectory = pak_header.read_binary(&mut buffer).unwrap();

        for (d, files) in &dir.files["vmt"] {
            println!("DIR: {}", d);
            for (_file, data) in files {
                let Ok(vmt) = data.load_vmt(&dir) else {
                    continue;
                };
                println!("{:?}", vmt.data);
            }
        }
    }
}
// export interface VMT {
//     _Root: string;
//     _Filename: string;

//     // patch
//     include: string;
//     replace: any;
//     insert: any;

//     // proxies
//     proxies: any;

//     // generic
//     [k: string]: VKFPairUnit;
// }

// export class ValveKeyValueParser {
//     private pos = 0;

//     constructor(private S: string) {
//     }

//     public hastok() {
//         return (this.pos < this.S.length);
//     }

//     public skipwhite(): void {
//         while (this.hastok()) {
//             const tok = this.S.charAt(this.pos);
//             if (/\s|[`\0]/.test(tok))
//                 this.pos++;
//             else
//                 return;
//         }
//     }

//     private skipcomment2(): boolean {
//         if (this.chew() === '/') {
//             const ch = this.chew(true);
//             if (ch === '/') {
//                 while (this.chew(true) !== '\n')
//                     ;
//                 return true;
//             } else if (ch === '*') {
//                 this.pos = this.S.indexOf('*/', this.pos) + 2;
//                 return true;
//             } else {
//                 throw "whoops";
//             }
//         } else {
//             this.spit();
//             return false;
//         }
//     }

//     private skipcomment(): void {
//         while (this.skipcomment2()) ;
//     }

//     private chew(white: boolean = false) {
//         if (!white)
//             this.skipwhite();
//         return this.S.charAt(this.pos++);
//     }

//     private spit(): void {
//         this.pos--;
//     }

//     private obj(): VKFPair[] {
//         // already consumed "{"
//         const val: VKFPair[] = [];
//         while (this.hastok()) {
//             this.skipcomment();
//             const tok = this.chew();
//             if (tok === "}" || tok === "") {
//                 return val;
//             } else {
//                 this.spit();
//             }

//             val.push(this.pair());
//         }
//         return val;
//     }

//     private quote(delim: string): string {
//         // already consumed delim
//         let val = "";
//         while (this.hastok()) {
//             const tok = this.chew(true);
//             if (tok == delim)
//                 return val;
//             else
//                 val += tok;
//         }
//         debugger;
//         throw "whoops";
//     }

//     private run(t: RegExp, start: string): string {
//         let val = start;
//         while (this.hastok()) {
//             const tok = this.chew(true);
//             if (t.test(tok)) {
//                 val += tok;
//             } else {
//                 this.spit();
//                 break;
//             }
//         }
//         return val;
//     }

//     private num(start: string): string {
//         const num = this.run(/[0-9.]/, start);
//         // numbers can have garbage at the end of them. this is ugly...
//         // shoutouts to materials/models/props_lab/printout_sheet.vmt which has a random letter "y" after a number
//         this.run(/[a-zA-Z]/, '');
//         return num;
//     }

//     private unquote(start: string): string {
//         return this.run(/[0-9a-zA-Z$%<>=/\\_,]/, start);
//     }

//     public unit(): VKFPairUnit {
//         this.skipcomment();

//         const tok = this.chew();
//         if (tok === '{')
//             return this.obj();
//         else if (tok === '"')
//             return this.quote(tok);
//         else if (/[a-zA-Z$%<>=/\\_,]/.test(tok))
//             return this.unquote(tok);
//         else if (/[-0-9.]/.test(tok))
//             return this.num(tok);
//         console.log(tok);
//         debugger;
//         throw "whoops";
//     }

//     public pair<T extends VKFPairUnit>(): VKFPair<T> {
//         const kk = this.unit();
//         if (typeof kk !== 'string') debugger;
//         const k = (kk as string).toLowerCase();
//         const v = this.unit() as T;
//         return [k, v];
//     }
// }

// function convertPairsToObj(o: any, pairs: VKFPair[], recurse: boolean = false, supportsMultiple: boolean = true): void {
//     for (let i = 0; i < pairs.length; i++) {
//         const [k, v] = pairs[i];
//         const vv = (recurse && typeof v === 'object') ? pairs2obj(v) : v;

//         if (k in o) {
//             if (supportsMultiple) {
//                 if (!Array.isArray(o[k]))
//                     o[k] = [o[k]];
//                 o[k].push(vv);
//             } else {
//                 // Take the first one.
//                 continue;
//             }
//         } else {
//             o[k] = vv;
//         }
//     }
//     return o;
// }

// export function pairs2obj(pairs: VKFPair[], recurse: boolean = false): any {
//     const o: any = {};
//     convertPairsToObj(o, pairs, recurse);
//     return o;
// }

// function patch(dst: any, srcpair: VKFPair[] | null, replace: boolean): void {
//     if (srcpair === null)
//         return;

//     for (const [key, value] of srcpair) {
//         if (key in dst || !replace) {
//             if (typeof value === 'object')
//                 patch(dst[key], value, replace);
//             else
//                 dst[key] = value;
//         }
//     }
// }

// function stealPair(pairs: VKFPair[], name: string): VKFPair | null {
//     const pair = pairs.find((pair) => pair[0] === name);
//     if (pair === undefined)
//         return null;

//     arrayRemove(pairs, pair);
//     return pair;
// }

// export async function parseVMT(filesystem: SourceFileSystem, path: string, depth: number = 0): Promise<VMT> {
//     async function parsePath(path: string): Promise<VMT> {
//         path = filesystem.resolvePath(path, '.vmt');
//         if (!filesystem.hasEntry(path)) {
//             // Amazingly, the material could be in materials/materials/, like is
//             //    materials/materials/nature/2/blenddirttojunglegrass002b.vmt
//             // from cp_mossrock
//             path = `materials/${path}`;
//         }
//         if (!filesystem.hasEntry(path))
//             path = `materials/editor/obsolete.vmt`;
//         const buffer = assertExists(await filesystem.fetchFileData(path));
//         const str = new TextDecoder('utf8').decode(buffer.createTypedArray(Uint8Array));

//         // The data that comes out of the parser is a nested series of VKFPairs.
//         const [rootK, rootObj] = new ValveKeyValueParser(str).pair<VKFPair[]>();

//         // Start building our VMT.
//         const vmt = {} as VMT;
//         vmt._Root = rootK;
//         vmt._Filename = path;

//         // First, handle proxies if they exist as special, since there can be multiple keys with the same name.
//         const proxiesPairs = stealPair(rootObj, 'proxies');
//         if (proxiesPairs !== null) {
//             const proxies = (proxiesPairs[1] as VKFPair[]).map(([name, value]) => {
//                 return [name, pairs2obj((value as VKFPair[]), true)];
//             });
//             vmt.proxies = proxies;
//         }

//         // Pull out replace / insert patching.
//         const replace = stealPair(rootObj, 'replace');
//         const insert = stealPair(rootObj, 'insert');

//         // Now go through and convert all the other pairs. Note that if we encounter duplicates, we drop, rather
//         // than convert to a list.
//         const recurse = true, supportsMultiple = false;
//         convertPairsToObj(vmt, rootObj, recurse, supportsMultiple);

//         vmt.replace = replace !== null ? replace[1] : null;
//         vmt.insert = insert !== null ? insert[1] : null;
//         return vmt;
//     }

//     const vmt = await parsePath(path);
//     if (vmt._Root === 'patch') {
//         const base = await parseVMT(filesystem, vmt['include'], depth++);
//         patch(base, vmt.replace, true);
//         patch(base, vmt.insert, false);
//         base._Patch = base._Filename;
//         base._Filename = vmt._Filename;
//         return base;
//     } else {
//         return vmt;
//     }
// }

// export function vmtParseVector(S: string): number[] {
//     // There are two syntaxes for vectors: [1.0 1.0 1.0] and {255 255 255}. These should both represent white.
//     // In practice, combine_tower01b.vmt has "[.25 .25 .25}", so the starting delimeter is all that matters.
//     // And factory_metal_floor001a.vmt has ".125 .125 .125" so I guess the square brackets are just decoration??

//     const scale = S.startsWith('{') ? 1/255.0 : 1;
//     S = S.replace(/[\[\]{}]/g, '').trim(); // Trim off all the brackets.
//     return S.split(/\s+/).map((item) => Number(item) * scale);
// }

// export function vmtParseColor(dst: Color, S: string): void {
//     const v = vmtParseVector(S);
//     dst.r = v[0] / 255.0;
//     dst.g = v[1] / 255.0;
//     dst.b = v[2] / 255.0;
//     dst.a = (v[3] !== undefined) ? (v[3] / 255.0) : 1.0;
// }

// export function vmtParseNumber(S: string | undefined, fallback: number): number {
//     if (S !== undefined) {
//         const v = vmtParseVector(S);
//         if (v[0] !== undefined && !Number.isNaN(v[0]))
//             return v[0];
//     }
//     return fallback;
// }
