# ASCII Hexdump

Converts unrecognized bytes to hexadecimal for easier analysis. For example, here are several lines taken from the EDRDG kradfile opened in VS Code as-is:

```text
# Aug 2005 - added ��; replaced � with ��
# Jan 2006 - added �� to ��
```

After using this tool, those lines become the following:

```text
# Aug 2005 - added C0 C6 ; replaced E9 B5  with F3 EE 
# Jan 2006 - added B0 EC  to BA A3 
```

See below for an example invocation:

```text
ascii_hexdump --input ./kradfile --output ./kradfile_dump.txt
```