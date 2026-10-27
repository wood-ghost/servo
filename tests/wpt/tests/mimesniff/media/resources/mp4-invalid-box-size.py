def main(request, response):
    box_size = request.GET.first(b"box_size", b"12")
    if box_size == b"12":
        # Size 12, "ftyp", major brand "isom": smaller than the offset (16)
        # where compatible brands begin, but long enough to be MIME-sniffed.
        body = bytes.fromhex("0000000c6674797069736f6d")
    elif box_size == b"16":
        # Control: includes a minor version and an empty compatible-brand list.
        body = bytes.fromhex("000000106674797069736f6d00000000")
    else:
        return 400, [(b"Content-Type", b"text/plain")], b"Unsupported box size"
    headers = [
        (b"Content-Type", b"video/mp4"),
        (b"Content-Length", str(len(body)).encode("ascii")),
        (b"Cache-Control", b"no-store"),
    ]
    return 200, headers, body
