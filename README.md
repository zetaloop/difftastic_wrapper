# difftastic_wrapper

A simple wrapper for [difftastic](https://github.com/Wilfred/difftastic) that adds `+`/`-` symbols before line numbers to make the output more like unified diff and easier for LLMs to read.

## What it does

Instead of this:
```diff
example.js --- 1/3 --- JavaScript
31                `User ${this.id}: Logged in at ${this.lastLogin.toISOString()}`
32            );
33        }
34
35        getUserSummary() {
36            return `ID: ${this.id}, Name: ${this.name}, Email: ${this.email}, Active: ${this.isActive}`;
   35     getUserDetails() {
   36         return `ID: ${this.id}, Name: ${this.name}, Email: ${
   37             this.email
   38         }, Active: ${this.isActive}, Last Login: ${
   39             this.lastLogin ? this.lastLogin.toLocaleDateString() : "Never"
   40         }`;
   41     }
   42 }
   43
   44 const userDatabase = [
```

You get this:
```diff
example.js --- 1/3 --- JavaScript
 31                `User ${this.id}: Logged in at ${this.lastLogin.toISOString()}`
 32            );
 33        }
 34
-35        getUserSummary() {
-36            return `ID: ${this.id}, Name: ${this.name}, Email: ${this.email}, Active: ${this.isActive}`;
+   35     getUserDetails() {
+   36         return `ID: ${this.id}, Name: ${this.name}, Email: ${
+   37             this.email
+   38         }, Active: ${this.isActive}, Last Login: ${
+   39             this.lastLogin ? this.lastLogin.toLocaleDateString() : "Never"
+   40         }`;
    41     }
    42 }
    43
    44 const userDatabase = [
```

## Installation

You need [difftastic](https://github.com/Wilfred/difftastic) installed first.

Then grab the binary from [releases](https://github.com/zetaloop/difftastic_wrapper/releases), or build it yourself:
```bash
cargo build --release
```

## Usage

Simply use `difftw` as a drop-in replacement for `difft`:
```bash
difftw file1.txt file2.txt
```

The wrapper automatically adds the `--display=inline` argument and forces `difft`
to produce colorized output (`--color=always`) so that the wrapper can parse it.

The decision to display colors in the final output follows the standard behavior
of `difft` itself. You can control this with the `--color` option (`always`,
`auto`, or `never`) or the `DFT_COLOR` environment variable. By default, the
behavior is `auto` (i.e., colors are enabled when writing to a terminal).

Examples:
```bash
# Enable colors (default when writing to a terminal)
difftw --color=always file1.txt file2.txt

# Disable colors
difftw --color=never file1.txt file2.txt

# Auto-detect based on whether output is a terminal (default)
difftw file1.txt file2.txt
```

**Note**: This tool only supports inline display mode. If you specify a different
value for `--display`, the tool will exit with an error.

For git integration:
```bash
git config --global diff.external "difftw"
```

This is a very experimental tool made for a specific use case - it only works with inline mode and relies on color detection.

## License

Unlicensed.
