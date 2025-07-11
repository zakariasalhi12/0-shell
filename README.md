# 0-shell

## Simple Shell Cycle Example
```rust
let shell = shell::new(); 

loop {
    shell.print_prompt();   // Display the command prompt to the user
    shell.read_line();      // Read user input from standard input
    shell.parse();          // Parse the input into commands and arguments
    shell.execute();        // Execute the parsed command
}
```

## ref
- https://gist.github.com/fnky/458719343aabd01cfb17a3a4f7296797
- https://www.youtube.com/watch?v=DqE6DxqJg8Q&t=322s
- https://en.wikipedia.org/wiki/Terminal_mode

