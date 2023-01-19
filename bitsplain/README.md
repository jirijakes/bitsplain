# Bitsplain

## About

Bitsplain is a library that helps people understand Bitcoin-related
binary data. When provided with some data — be it a binary file, hex-encoded
string or any other commonly used encoding scheme — Bitsplain first tries to
identify what the data represent and if it succeeds it offers an explanation
of the data through “annotations”. These annotations consist of description, data type,
rendered value, position in the binary input etc.

The library does not interpret the annotations, however crate `bitsplain-bin`
offers two user interfaces, a CLI and GTK.
