# opvm2

A better version of opvm.

## How to use
To use opvm2, create a new `.o2` file, and write your code into it.

### Interpreting
Then, run the following command:
```bash
./target/release/opvm2_cli -i --file file.o2
```
This will interpret your code on the fly.

### Compiling
To compile your code, run the following command:
```bash
./target/release/opvm2_cli -c --file file.o2
```
Then you can run the compiled binary with:
```bash
./target/release/opvm2_cli --file file.o2c
```

## Opcode List
| Opcode | Description |
|--------|-------------|
| mov    | Move a value into a register |
| add    | Add two values together |
| sub    | Subtract two values |
| mul    | Multiply two values |
| div    | Divide two values |
| mod    | Modulo two values |
| xor    | XOR two values |
| inc    | Increment a register |
| dec    | Decrement a register |
| jmp    | Jump to an address |
| jl     | Jump if less than |
| jg     | Jump if greater than |
| je     | Jump if equal |
| jne    | Jump if not equal |
| jle    | Jump if less than or equal |
| jge    | Jump if greater than or equal |
| jz     | Jump if zero |
| jnz    | Jump if not zero |
| test   | Test two values |
| push   | Push a value onto the stack |
| pop    | Pop a value off the stack |
| call   | Call a function |
| ret    | Return from a function |
| halt   | Halt the program |
| nop    | No operation |
| assert | Assert a value |
| print  | Print a value |
| sleep  | Sleep for a number of milliseconds |

## Syntax
The syntax is very simple. Each line is a new instruction.
The first word is the opcode, then the arguments follow as operands.
For example:
```asm
mov r0, 0
```
This instruction will move the value `0` into register `r0`.

```asm
jmp r0
```
This instruction will jump to the address stored in register `r0`.

You also have the ability to have labels and literals.
```asm
; literal example
myString 'Hello, World!'
```
This will create a string in memory with the value `Hello, World!`.

```asm
; label example
loop:
    inc ra
    test ra, 10
    jl loop
```
This will increment register `ra` until it reaches `10`.

## Debugger
OPVM2 comes with a purpose built debugger to help you step through your code and see what's happening. To use the debugger, simply run your code with the `-d` flag.
```bash
./target/release/opvm2_cli -d --file file.o2c
```


## Plugins
OPVM2 has a plugin system that allows you to extend the functionality of the VM. To create a plugin, look at the `plugin_test` project.

### What are plugins for?
Plugins are used to extend the functionality of the VM. For example, you could create a plugin that adds a new opcode to the VM, or a new function that can be called from the VM.

Look at `plugin_test` for an example of how to create a plugin and use it in your code to make a new opcode.

### Using plugins
To use a plugin, you need to compile it and then load it into the VM. To load a plugin, use the `-p` flag.
```bash
./target/release/opvm2_cli -c --file plugin.o2 -p path/to/plugin.wasm
```
When compiling your code with plugins, it will embed the plugin into the final binary, so you don't need to pass it in when running a compiled binary.
```bash
./target/release/opvm2_cli --file plugin.o2c
```

## Building
To build the project, run the following command:
```bash
#!/bin/zsh
cargo build --package debugger --target wasm32-unknown-unknown --release
cargo build --package opvm2_cli --release
# Any additional plugins
cargo build --package plugin_test --target wasm32-unknown-unknown --release
```

## Examples
Here's an example of the infamous "FizzBuzz" program:
```asm
fizz: 'Fizz'             ; define fizz literal
buzz: 'Buzz'             ; define buzz literal

       mov ra, 1         ; move 1 into ra
start: call calc_fizz
       call calc_buzz
       call print_number
       print_ascii 10    ; print newline
       inc ra            ; increment ra by 1
       test ra, 20       ; check if we've looped less than 20 times
       jle start         ; if so, jump back to start
       jmp end           ; otherwise, jump to end
print_number:
       pop rc            ; check if buzz was printed
       test rc, rc
       jnz skip          ; if it was, skip printing the number
       pop rc            ; check if fizz was printed
       test rc, rc
       jnz skip          ; if it was, skip printing the number
       print ra          ; print number
skip:  ret
calc_fizz:
       mov rb, ra        ; move ra into rb, temp value cloning
       mod rb, 3         ; modulo rb by 3
       test rb, rb       ; find out if the modulo result is zero
       jnz skip_fizz     ; if not, jump to skip_fizz
       print_ascii fizz  ; print fizz
       push 1            ; push 1 onto stack to indicate fizz was printed
       ret
skip_fizz:
       push 0            ; push 0 onto stack to indicate fizz was not printed
       ret
calc_buzz:
       mov rb, ra        ; move ra into rb, temp value cloning
       mod rb, 5         ; modulo rb by 5
       test rb, rb       ; find out if the modulo result is zero
       jnz skip_buzz     ; if not, jump to skip_buzz
       print_ascii buzz  ; print buzz
       push 1            ; push 1 onto stack to indicate buzz was printed
       ret
skip_buzz:
       push 0            ; push 0 onto stack to indicate buzz was not printed
       ret
end:
```
Output:
```
1
2
Fizz
4
Buzz
Fizz
7
8
Fizz
Buzz
11
Fizz
13
14
FizzBuzz
16
17
Fizz
19
Buzz
```
