# discord-chat-stats

A simple tool to extract discord statistics from chat logs.

We can currently do:
* The distribution of contributions on the server (e.g. how many users are
  really active) (`--dist`).
* Overall word count breakdown (`--count`).
* Specific word count breakdown (`--word <word>`).

Note: You'll need to export the chat data into a folder called `exports`.

How you do this, is up to you, but it needs to contain CSV files with the
following structure:

```
AuthorID,Author,Date,Content,Attachments,Reactions
"456226577798135808","Deleted User#0000","01-Jan-20 01:33 AM","Hello World","",""
...
```

And so forth.

Once you have this, you can run the tool using:

```
cargo run -- <args>
```

For a bit more help, try:

```
cargo run -- --help
```
