import hid
import time

def to_hex(data):
    return " ".join(f"{x:02x}" for x in data)

def deep_scan():
    path = b'/dev/hidraw6'
    dev = hid.device()
    dev.open_path(path)
    dev.set_nonblocking(False)

    print("🧬 Mapping MX Vertical Internal DNA...")
    print(f"{'Slot':<6} | {'Feature':<10} | {'Status':<20}")
    print("-" * 45)

    for slot in range(1, 0x30): # Scanning up to 48 slots
        # Query Feature ID for this slot
        # Feature 0x0001 Function 0x01 = Get Feature ID
        msg = [0x11, 0x01, 0x01, 0x01, slot] + [0x00]*15
        dev.write(msg)
        res = dev.read(20, timeout_ms=50)
        
        if res and res[2] != 0xff and len(res) > 5:
            feature_id = (res[4] << 8) | res[5]
            
            # Now, if it's a known sensor feature, try to READ it
            # 0x2201 = Adjustable DPI
            # 0x2121 = Hi-Res Reporting
            status = "ID Found"
            if feature_id == 0x2201:
                status = "🎯 TARGET: DPI"
            elif feature_id == 0x1000:
                status = "🔋 TARGET: Battery"
                
            print(f"0x{slot:02x}   | 0x{feature_id:04x}   | {status}")
            
    dev.close()

if __name__ == "__main__":
    deep_scan()
