import sys

def encrypt(plaintext: str) -> str:
    data = plaintext.encode('utf-8')
    state = 0x8F3C  # 16-bit internal seed
    tokens = []
    
    for i, byte in enumerate(data):
        # Generate 4 pseudo-random sub-keys derived from state & position
        k1 = (state + i * 13) % 256
        k2 = (state * 7 + i * 31) % 256
        k3 = (state ^ (i * 101)) % 256
        k4 = ((state >> 3) + i * 17) % 256
        
        # Split byte across 4 encrypted tokens
        t1 = (byte + k1) % 256
        t2 = (byte ^ k2) % 256
        t3 = (byte + k3 + i) % 256
        t4 = (byte ^ k4 ^ (i * 3)) % 256
        
        tokens.extend([f"{t1:03d}", f"{t2:03d}", f"{t3:03d}", f"{t4:03d}"])
        
        # Evolve internal state dynamically
        state = (state * 251 + byte + i * 9) % 65536

    return "_".join(tokens)

if __name__ == "__main__":
    msg = input("Enter text to encrypt: ")
    if msg:
        print("\nEncrypted Data:\n" + encrypt(msg))

envi = input("$ ")
if envi == "1":
    pass