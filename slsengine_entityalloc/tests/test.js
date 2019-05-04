import {
  default as wasm,
  GenerationalIndexAllocator,
  JsIndexArray
} from '/pkg/slsengine_entityalloc.js';

function test_assert(cond, ...message) {
  if (!cond) {
    console.error('test failed: ', ...message);
  }
}

async function run() {
  await wasm('/pkg/slsengine_entityalloc_bg.wasm');
  let allocator = new GenerationalIndexAllocator(10);
  let index = new JsIndexArray();
  let a = allocator.allocate();
  let b = allocator.allocate();
  let c = allocator.allocate();

  try {
    let obj = { name : 'an_object' };

    index.insert(a, 10);
    index.insert(b, obj);

    test_assert(index.get(a) === 10, 'index at a should be 10');
    test_assert(index.get(b).name === 'an_object',
                `index at b should be {obj}`);
    test_assert(index.remove(a) === 10, 'removing object should return it');
    test_assert(!index.get(a), 'remove should remove it');

  } finally {
    for (let i of [allocator]) {
      i.free();
    }
  }
}

run();