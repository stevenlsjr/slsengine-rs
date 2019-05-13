
import('../pkg/slsengine_entityalloc').then(async ({GenerationalIndexAllocator, JsIndexArray, ...mod} ) =>{
    let alloc = new GenerationalIndexAllocator(255);
    let arr = new JsIndexArray();
    let idx = alloc.allocate();
    arr.set(idx, 'foo');
    let val = arr.get(idx);
   console.log(val);
   arr.remove(idx);
   console.log(arr.get(idx));

    try {

    } finally {
        for (let i of [alloc, arr]) {
            i.free();
        }
    }




});