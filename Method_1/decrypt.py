while True:
    text = input("Text to decrypt (q to quit): ")
    if text == "q":
        break

    state = 17
    out = bytearray()

    for i, token in enumerate(text.split("_")):
        x = int(token)
        b = (x - state - i * 7) % 256
        out.append(b)
        state = (state * 31 + b + i) % 256

    print("Decrypted:", out.decode())
