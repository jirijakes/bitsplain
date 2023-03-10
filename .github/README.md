# Bitsplain

## About

Bitsplain is a library that helps people understand Bitcoin-related
binary data. When provided with some data — be it a binary file, hex-encoded
string or any other commonly used encoding scheme — Bitsplain first tries to
identify what the data represent and if it succeeds it offers an explanation
of the data through “annotations”. These annotations consist of description, data type,
rendered value, position in the binary input etc.

The library does not interpret the annotations, however crate
[bitsplain-bin](https://crates.io/crates/bitsplain-bin) offers two user interfaces, a CLI and GTK.

## Status

Under heavy development, not yet ready for being used.

Library decodes binary input by trying multiple parsers and will return annotations for
the successful ones. So far it can detect only a few kinds of data (block header,
transaction, some LN gossip messages and a few more). The annotations are not yet complete
and too detailed.

More parsers and more annotations will be continuously added.

## Can it be re-used for non-Bitcoin purposes?

Not now but it may be possible at some point.

## Try

```
fossil clone https://jirijakes.com/code/bitsplain
cd bitsplain
cargo install --path bitsplain-bin

bitsplain DATA

bitsplain-gtk DATA
```

Instead of `DATA`, you can pass one of these. You can provide them as arguments or paste them (`CTRL+V`) into running GTK application.

`02000000012ee9bfce8f056cd097bcaec5a0c748ea76d6a3ed68d0e9939feb200b64816788040000008b483045022100a30ad9317573aa3d6e9c668bc510e6a828261bc2db08f2fb03de901a3fb9370e02204396fd8c14ab2e0355981c7a88da103f874e47d49bf3148180b061568a5669070141047146f0e0fcb3139947cf0beb870fe251930ca10d4545793d31033e801b5219abf56c11a3cf3406ca590e4c14b0dab749d20862b3adc4709153c280c2a78be10cffffffff036f240c000000000017a914f82921cc8545c477bb9c4d60c9d6b097299d278787aeef1200000000001976a9149b0493f8c16f9e9b4f288cfd27b753b296e395f488ac5eb92e52200000001976a91443849383122ebb8a28268a89700c9f723663b5b888ac00000000`

`0000002045569767d88d3962a3fbb9a0776f1f97c636f327d8ec0300000000000000000026ed8403a595b6e5fd172050db42eabfe76700aedd17e6b2ce416b92a1c4b7660553aa5d5ca31517a7ad933f`

`lno1pqqkgz3zxycrqmtnv96zqetkv4e8jgryv9ujcgrxwfhk6gp3949xzm3dxgcryvg5zpe82um50yhx77nvv938xtn0wfn35qspqywq2q2laenqq83qfwdpl28qqmc78ymlvhmxcsywdk5wrjnj36jryg488qwlrnzyjczlqsytf0g06zjgcrtk0n09n5wk78ssdhckpmfqmfvlxm92u36egsmf3kswfpqt70dq6mg4lw3t8qx7feh6c8hxz2vwzsdg4n957z8gh8unx`

`PM8TJTLJbPRGxSbc8EJi42Wrr6QbNSaSSVJ5Y3E4pbCYiTHUskHg13935Ubb7q8tx9GVbh2UuRnBc3WSyJHhUrw8KhprKnn9eDznYGieTzFcwQRya4GA`

`01002e8faaa0fd5119e595949386f3f2f48090ceebb7048335327eaaf402b522ef4c4a60abd477f6a98395d4ab8c30d0e94d05f89200a36713b603ffd51effc007481e2a7e5e31cf4e51ce39fd8268bd3cfd63afb9d7a3ae23789b9d5a5250a1413d5aa7e78ad7536fe313a50b9e0db2cc786cdb134b6a5b3a2ec16a63a15290cfc6585c55cbe42670a2bdc74d663d38eb894243194b6fd112d3f8e6a57f1d10d2c30700092553319f51d76e1446674be4a65bd696638c7a8983ca09b5a94e35cbd53dceaac31298d3be8ca2db7214a4641079a2ef7406740614eb4e3f1c5bb20fda0302c4089fd3b15b8027ff201c475010932b281ec73f9cf83f9816424aad5342000006226e46111a0b59caaf126043eb5bbf28c34f3a5e332a1fc7b2b73cf188910f00006c000001000102c5c74c58f37aedb64886d0345f732e45d3c9789216913a93a4ab7853dc4e8b0a039d33009dc6b3e36bb1915a0f1aeb6d10e412a9b8bc818828746733fef204732702b6be0f40f167e9c0ee495cca63904fbf06cc4d4576df68dd929cf678b13aada00272fa7dcb25e15bae639497920116ec7c47251fcf947b4bbd0f3548b210e88440`


## Screenshots of user interfaces (bitsplain-bin)

### CLI

![](https://jirijakes.com/code/bitsplain/raw/bfb061b1324d8dc2d2929fc62e15301169bd31d897fa3af6c14448ab79464a90?m=image/png)

### GTK

![](https://jirijakes.com/code/bitsplain/raw/4d6be69ca43a291cd1925555d2d93c5ca94f39f9e662b18c2b80c1868a8c4991?m=image/png)
