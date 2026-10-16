import secrets

# 32 bytes produce 64 hexadecimal characters
hex_string = secrets.token_hex(32)
print(hex_string)