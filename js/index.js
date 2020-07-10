import("../pkg/index.js").then((pkg) => {
  pkg.wasm_hello('bloop');
}).catch(console.error);

