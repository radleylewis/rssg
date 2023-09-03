<div align="center">
 <img src="https://github.com/radleylewis/rssg/assets/40852773/5bef91b6-10f3-425e-b3d7-52414faca447" width="250px">
 <p>A Static Site Generator built in Rust.</p>
</div> 

## Introduction

Write your new static site with Markdown or HTML. This Static Site Generator includes:

- CSS theme with built in dark/light theme
- zero JavaScript
- support for HTML or Markdown

## Preview

![output](https://github.com/radleylewis/rssg/assets/40852773/e4466739-6104-41bd-8e0f-0be2d056102c)

## Usage

To get started, first compile the binary:

```bash
cargo build --release
```

Use your rssg binary to initialise your new project with:

```bash
rssg init
```

Complete the prompts and you will have a new project (default is **my-project**):

From your new project directory build your project to create your **dist** folder to deploy your website:

```bash
cd my-project
rssg build  
```

## Author

Radley E. Sidwell-Lewis
