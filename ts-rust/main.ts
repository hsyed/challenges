const libPath = "./target/release/libts_rust.so";

const lib = Deno.dlopen(libPath, {
  add_numbers: {
    parameters: ["i32", "i32"],
    result: "i32",
  },
  greet: {
    parameters: ["pointer"],
    result: "pointer",
  },
  free_string: {
    parameters: ["pointer"],
    result: "void",
  },
  fibonacci: {
    parameters: ["u32"],
    result: "u64",
  },
  sum_array: {
    parameters: ["pointer", "usize"],
    result: "i32",
  },
});

function greetFromRust(name: string): string {
  const nameBuffer = new TextEncoder().encode(name + "\0");
  const namePtr = Deno.UnsafePointer.of(nameBuffer);

  const resultPtr = lib.symbols.greet(namePtr);
  const result = new Deno.UnsafePointerView(resultPtr!).getCString();

  lib.symbols.free_string(resultPtr);

  return result;
}

function sumArrayInRust(numbers: number[]): number {
  const buffer = new Int32Array(numbers);
  const ptr = Deno.UnsafePointer.of(buffer);

  return lib.symbols.sum_array(ptr, BigInt(numbers.length)) as number;
}


function main() {
  console.log("🦀 Rust + TypeScript + Deno Example");
  console.log("====================================");

  console.log(`Adding 15 + 27: ${lib.symbols.add_numbers(15, 27)}`);

  console.log(`Greeting: ${greetFromRust("Deno Developer")}`);

  console.log(`Fibonacci of 10: ${lib.symbols.fibonacci(10)}`);

  const numbers = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
  console.log(`Sum of [${numbers.join(", ")}]: ${sumArrayInRust(numbers)}`);

  console.log("\n✨ All functions called successfully!");
}

if (import.meta.main) {
  main();
}
