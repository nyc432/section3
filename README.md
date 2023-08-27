

# TABLE OF CONTENTS

* ## [USE](#use)

* ## [FUTURE](#future)



------------------------------

# USE {#use}

## Blog Posts  

Go to the make_blog directory and do 

    cargo run

Blog posts go in blog_content. Names should be in the format

    yyyy-mm-ddthh-mm.md

Why the "t" you may ask? Because if there's a - or _ there, it's treated as a date and
that causes craziness with normal Zola processing. It's normally good but I'm
using my Rust code to create the blog. And my Rust parsing has to assume something
in particular there, but I could find no non-alphanumeric characters 
that would be easy to handle for both a filename and a URL. 


    zola serve

it isn't reachable at the URL. I have absolutely no idea why this would be the case. It's bizarre but it appears to be the way it is.


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

# FUTURE  {#future}
Probably if I look at the introductory example for Zola, which has a blog example, I can get rid off all my Rust code and do it that way. But the Rust code also means I have a way of making my own blog in other contexts.