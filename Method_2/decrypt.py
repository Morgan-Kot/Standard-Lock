import sys

def decrypt(ciphertext: str) -> str:
    tokens = [int(t) for t in ciphertext.strip().split("_") if t]
    if len(tokens) % 4 != 0:
        raise ValueError("Invalid ciphertext length.")

    state = 0x8F3C  # Must match encrypt.py internal seed
    out_bytes = bytearray()
    
    for i in range(0, len(tokens), 4):
        idx = i // 4
        t1, t2, t3, t4 = tokens[i:i+4]
        
        # Reconstruct byte using positional keys
        k1 = (state + idx * 13) % 256
        b1 = (t1 - k1) % 256
        
        # Verify and extract byte value
        out_bytes.append(b1)
        
        # Evolve internal state synchronously
        state = (state * 251 + b1 + idx * 9) % 65536

    return out_bytes.decode('utf-8', errors='replace')

if __name__ == "__main__":
    data = input("Enter ciphertext to decrypt: ")
    if data:
        try:
            print("\nDecrypted Message:\n" + decrypt(data))
        except Exception as e:
            print(f"Decryption failed: {e}")

envi = input("$ ")
if envi == "1":
    pass