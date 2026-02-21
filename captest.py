import hid
import time

def to_hex(data):
    return "".join(f"{x:02x}" for x in data)

def find_mouse_brain():
    # Identify the Receiver
    vendor_id = 0x046d
    product_id = 0xc52b
    
    print("🚀 Initializing MX Vertical Manager...")
    
    for i in range(10): # Check first 10 hidraw nodes
        path = f"/dev/hidraw{i}"
        try:
            dev = hid.device()
            dev.open_path(path.encode())
            dev.set_nonblocking(False)
            
            # We try Device Index 0x01 (First paired device)
            # Query: Feature 0x0000 (Root), Function 0x00 (Get Feature), Data: 0x0001 (Feature Set)
            # Using Short Report (0x10)
            msg = [0x10, 0x01, 0x00, 0x00, 0x00, 0x01, 0x00]
            dev.write(msg)
            
            res = dev.read(20, timeout_ms=200)
            if res and res[0] == 0x10 and res[4] != 0:
                print(f"✅ Found Mouse on {path} (Device Index 0x01)")
                feature_set_idx = res[4]
                
                # Now find the DPI feature (0x2001)
                # Query: Root, Get Feature, Data: 0x2001
                dpi_query = [0x10, 0x01, 0x00, 0x00, 0x20, 0x01, 0x00]
                dev.write(dpi_query)
                dpi_res = dev.read(20, timeout_ms=200)
                
                if dpi_res and dpi_res[4] != 0:
                    print(f"🎯 DPI Control found at Feature Index: {hex(dpi_res[4])}")
                    return dev, dpi_res[4]
            
            dev.close()
        except:
            continue
    
    print("❌ Could not find an active HID++ 2.0 connection. Move your mouse!")
    return None, None

if __name__ == "__main__":
    device, dpi_idx = find_mouse_brain()
    if device:
        # Success! Now we can build the 'Set DPI' function.
        device.close()
