# WebGL Calculator

This project is a graphical calculator made with Tauri that draws the functions using the fragment shader with WebGL2.

It has:
- Graph almost any 2d equation
- Use of one-letter variables and functions
- Derivatives of the functions

![Screenshot of the app](src/assets/screenshot.png)

## How it works
The user inputs a equation, like in Desmos, an it is send to the backend written in LaTeX, made in Rust. 
Then the Rust Backend compiles the equation and it sends it back to the TS frontend, were it is send to the graphics card, where it is interpreted and graphed in the fragment shader.

## Tecnologies used
- Tauri
- Rust
- Typescript
- WebGL2
- MathQuill JS

## How to run
Execute the command:
```console
npm run tauri dev 
```