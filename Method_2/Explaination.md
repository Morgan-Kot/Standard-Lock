### Cipher Formulas

This cipher uses an autokey feedback loop where each character's output depends on its position and the preceding decrypted byte.

**Encryption:**
$$x_i = (b_i + \text{state}_i + 7i) \bmod 256$$

**Decryption:**
$$b_i = (x_i - \text{state}_i - 7i) \bmod 256$$

**State Evolution:**
$$\text{state}_{i+1} = (31 \cdot \text{state}_i + b_i + i) \bmod 256 \quad \text{where } \text{state}_0 = 17$$