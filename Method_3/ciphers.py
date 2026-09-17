class BaseCipher:
    name = "Base"
    desc = "Base cipher class"

    @staticmethod
    def encrypt(plaintext: str) -> str:
        raise NotImplementedError

    @staticmethod
    def decrypt(ciphertext: str) -> str:
        raise NotImplementedError


class STWE01Cipher(BaseCipher):
    name = "STWE01 (Stewie)"
    desc = "Standard Two way Encryption No.1 - Tokenized feedback state encryption"

    @staticmethod
    def encrypt(plaintext: str) -> str:
        if not plaintext:
            return ""
        data = plaintext.encode('utf-8')
        state = 0x8F3C  # 16-bit internal seed
        tokens = []
        
        for i, byte in enumerate(data):
            k1 = (state + i * 13) % 256
            k2 = (state * 7 + i * 31) % 256
            k3 = (state ^ (i * 101)) % 256
            k4 = ((state >> 3) + i * 17) % 256
            
            t1 = (byte + k1) % 256
            t2 = (byte ^ k2) % 256
            t3 = (byte + k3 + i) % 256
            t4 = (byte ^ k4 ^ (i * 3)) % 256
            
            tokens.extend([f"{t1:03d}", f"{t2:03d}", f"{t3:03d}", f"{t4:03d}"])
            state = (state * 251 + byte + i * 9) % 65536

        return "_".join(tokens)

    @staticmethod
    def decrypt(ciphertext: str) -> str:
        if not ciphertext.strip():
            return ""
        tokens = [int(t) for t in ciphertext.strip().split("_") if t]
        if len(tokens) % 4 != 0:
            raise ValueError("Invalid ciphertext structure.")

        state = 0x8F3C
        out_bytes = bytearray()
        
        for i in range(0, len(tokens), 4):
            idx = i // 4
            t1 = tokens[i]
            
            k1 = (state + idx * 13) % 256
            b1 = (t1 - k1) % 256
            
            out_bytes.append(b1)
            state = (state * 251 + b1 + idx * 9) % 65536

        return out_bytes.decode('utf-8', errors='replace')


class CaesarCipher(BaseCipher):
    name = "Caesar (Shift +3)"
    desc = "Classic Caesar Shift cipher moving characters by 3 positions."

    @staticmethod
    def encrypt(plaintext: str) -> str:
        res = []
        for char in plaintext:
            res.append(chr((ord(char) + 3) % 1114112))
        return "".join(res)

    @staticmethod
    def decrypt(ciphertext: str) -> str:
        res = []
        for char in ciphertext:
            res.append(chr((ord(char) - 3) % 1114112))
        return "".join(res)


# Registry list of all available ciphers
CIPHERS = {
    STWE01Cipher.name: STWE01Cipher,
    CaesarCipher.name: CaesarCipher,
}