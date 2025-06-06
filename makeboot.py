import sys

def assemble(instructions, outname="boot.bin"):
    boot = []

    # Parse instruction strings like ["EB", "FE", "90"]
    for byte_str in instructions:
        if len(byte_str) != 2:
            print(f"Invalid byte: '{byte_str}'")
            return
        try:
            value = int(byte_str, 16)
        except ValueError:
            print(f"Not a hex byte: '{byte_str}'")
            return
        boot.append(value)

    # Pad to 510 bytes
    while len(boot) < 510:
        boot.append(0x00)

    # Add signature at 510 and 511
    boot.append(0x55)
    boot.append(0xAA)

    if len(boot) != 512:
        print("Something went wrong. Final size != 512 bytes.")
        return

    # Write it out
    with open(outname, "wb") as f:
        f.write(bytes(boot))

    print(f"Boot sector written to {outname}")

# Example usage
if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python makeboot.py <hex bytes...>")
        print("Example: python makeboot.py EB FE 90")
        sys.exit(1)

    instructions = sys.argv[1:]
    assemble(instructions)