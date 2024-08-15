import * as wasm from "bsp-web";

//wasm.greet();


document.getElementById("filepicker").addEventListener(
	"change",
	(event) => {
		let output = document.getElementById("listing");
		console.log(event.target.files);

		wasm.run_mesh(event.target.files, document.getElementById("canvas"));
		// wasm.open_folder(event.target.files);

		// for (const file of event.target.files) {
		// 	let item = document.createElement("li");
		// 	item.textContent = file.webkitRelativePath;


		// 	output.appendChild(item);
		// }
	},
	false,
);

document.getElementById("vpktest").addEventListener(
	"change",
	async (event) => {
		console.log(event.target.files[0]);

		await wasm.test_load_vpk(event.target.files[0]);

		await wasm.run_blank(document.getElementById("canvas"));
	},
	false,
);