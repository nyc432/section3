# Blog Posts

Go to the make_blog directory and do 

    cargo run

Blog posts go in blog_content. Names must be in the format

    yyyy-mm-ddthh-mm.md

Why the "t" you may ask? 


They must have frontmatter like

```
+++
title = "Some blog title"
weight = 1
+++
```

The weight is ignored but if it's not there, there's an error message from zola serve
and I'm afraid there could be an error after deployment.

Filenames must be in the format yyyy-mm-dd-hh-mm.md. I tried to use an underscore or a space between the date and the time. They didn't work. Zola turns the underscores into dashes for its internal links. I could code it to use underscores for the file names and dashes in the urls but that would probably be more confusing than it's worth. Space didn't work either but I haven't studied why.