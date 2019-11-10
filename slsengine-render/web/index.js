const root = document.createElement("div");


document.body.appendChild(root);
import('../pkg').then(rust =>{
  window.WASM=rust;
  rust.sample_main(root)
})