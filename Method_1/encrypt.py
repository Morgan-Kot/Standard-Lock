while True:
    text = input("Text to encrypt (q to quit): ")
    if text == "q":
        break

    state = 17
    out = []

    for i, b in enumerate(text.encode()):
        x = (b + state + i * 7) % 256
        out.append(f"{x:03}")
        state = (state * 31 + b + i) % 256

    print("Encrypted:", "_".join(out))
