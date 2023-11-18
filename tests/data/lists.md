# Test Markdown

## Lists

- One
- Two
- Three
  - Three-One
  - Three-Two
    - Three-Two-One
    - Three-Two-Two
  - Three-Three
- Four
- Five
  - Five-One
  - Five-Two

1. One
2. Two
3. Three
    1. Three-One
    2. Three-Two
        1. Three-Two-One
        2. Three-Two-Two
    3. Three-Three
4. Four
5. Five
  5.1 Five-One
  5.2 Five-Two

- one
- two

1.  This is a list item with two paragraphs. Lorem ipsum dolor
    sit amet, consectetuer adipiscing elit. Aliquam hendrerit
    mi posuere lectus.

    Vestibulum enim wisi, viverra nec, fringilla in, laoreet
    vitae, risus. Donec sit amet nisl. Aliquam semper ipsum
    sit amet velit.

2.  Suspendisse id sem consectetuer libero luctus adipiscing.


-  This is a list item with two paragraphs. Lorem ipsum dolor
    sit amet, consectetuer adipiscing elit. Aliquam hendrerit
    mi posuere lectus.

    Vestibulum enim wisi, viverra nec, fringilla in, laoreet
    vitae, risus. Donec sit amet nisl. Aliquam semper ipsum
    sit amet velit.

-  Suspendisse id sem consectetuer libero luctus adipiscing.
test data

## Code

```sh
cargo build
```

```sh
$ cargo run
Here is a message
```

## Inline code

Here is some `inline` code

## *Emphasis*

This text has *emphasis*, *indeed*

## **Strong**

This text is **strong**, like **iron**

## ~~Delete~~

This shall be ~~deleted~~, ~~where it is?~~

## Image

![alt text](https://th-thumbnailer.cdn-si-edu.com/ii_ZQzqzZgBKT6z9DVNhfPhZe5g=/fit-in/1600x0/filters:focal(1061x707:1062x708)/https://tf-cmsv2-smithsonianmag-media.s3.amazonaws.com/filer_public/55/95/55958815-3a8a-4032-ac7a-ff8c8ec8898a/gettyimages-1067956982.jpg "Title")

## Blockquote

> The first rule about fight club is you don’t talk about fight club.

Some text

> The second rule about fight club is you don’t talk about fight club.

## Thematic breaks

A paragraph before the thematic break.

* * *

A paragraph after the thematic break.

## Embedded HTML

<div>
  <img src="https://th-thumbnailer.cdn-si-edu.com/ii_ZQzqzZgBKT6z9DVNhfPhZe5g=/fit-in/1600x0/filters:focal(1061x707:1062x708)/https://tf-cmsv2-smithsonianmag-media.s3.amazonaws.com/filer_public/55/95/55958815-3a8a-4032-ac7a-ff8c8ec8898a/gettyimages-1067956982.jpg">
</div>

## Image Reference

![alpha][bravo]

## Definition

Open a [cat]

[cat]: https://th-thumbnailer.cdn-si-edu.com/ii_ZQzqzZgBKT6z9DVNhfPhZe5g=/fit-in/1600x0/filters:focal(1061x707:1062x708)/https://tf-cmsv2-smithsonianmag-media.s3.amazonaws.com/filer_public/55/95/55958815-3a8a-4032-ac7a-ff8c8ec8898a/gettyimages-1067956982.jpg

## Link/Footnote reference

Here is a simple footnote[^1]. With some additional text after it.

[^1]: My reference.

## Footnote definition

Using footnotes is fun![^1] They let you reference relevant information without disrupting the flow of what you’re trying to say.[^bignote]

[^1]: This is the first footnote.
[^bignote]: Here’s one with multiple paragraphs and code.

    Indent paragraphs to include them in the footnote.

    ```
    my code
    ```

    Add as many paragraphs as you like.

Text here and here and here.
[Learn more about markdown and footnotes in markdown](https://docs.github.com/en/github/writing-on-github/getting-started-with-writing-and-formatting-on-github/basic-writing-and-formatting-syntax#footnotes)

## Table

| Month    | Savings |
| :------- | ------- |
| January  | $250    |
| February | $80     |
| March    | $420    |

| Month    | Savings |
| -------- | ------- |
| January  | $250    |
| February | $80     |
| March    | $420    |

| Month    | Savings |
| :------: | ------: |
| January  | $250    |
| February | $80     |
| March    | $420    |

| Month    | Savings |
| January  | $250    |
| February | $80     |
| March    | $420    |
