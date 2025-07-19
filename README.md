# typistapp 

A CLI tool and Rust crate that generates Japanese ASCII art from images and displays it with a typing animation.

## Features

- Convert images into ASCII art using full-width Japanese characters
- Animate the ASCII art from top-left to bottom-right (typing effect)
- Usable both as a CLI tool and as a library crate

## Installation

```bash
cargo install --git https://github.com/anrinakamura/typistapp-rs
```

## Usage 

```bash
# Basic usage
typistapp <OUTPUT_WIDTH> --image <PATH_TO_IMAGE>

# Example: Generate an 80-character wide ASCII art from cat.png
typistapp 80 --image ./cat.png
```

| Argument/Option | Description |
| :--- | :--- |
| `<OUTPUT_WIDTH>` | (Required) The width of the output ASCII art in characters. Must be between 32 and 128. |
| `-i`, `--image` | (Required) The path to the image file you want to convert. |

## License

This project is licensed under the [MIT License](LICENSE).
