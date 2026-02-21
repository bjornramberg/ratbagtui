import hid
import time

def brute_force_indices():
    vendor_id = 0x046d
    product_id = 0xc52b
    
    # We'll try common hidraw nodes discovered earlier
    paths = [b'/dev/hidraw1', b'/dev/hidraw5', b'/dev/hidraw6']
    
    for path in paths:
        try:
            dev = hid.device()
            dev.open_path(path)
            dev.set_nonblocking(True)
            print(f"\n--- Scanning path: {path} ---")

            # HID++ 2.0 Get Feature Index for Root (0x0000)
            # We try every index from 0x01 to 0x06 (standard wireless slots)
            for index in range(1, 7):
                # Long Report (0x11), Index, Root(0x00), GetFeature(0x00), FeatureSet(0x0001)
                msg = [0x11, index, 0x00, 0x00, 0x00, 0x01] + [0x00]*14
                dev.write(msg)
                time.sleep(0.05)
                
                res = dev.read(20)
                if res:
                    print(f"  [!] RESPONSE from Index {hex(index)}: {' '.join(f'{x:02x}' for x in res)}")
                    # If we see a response starting with 11 [Index] 00 00, we found it!
                    if res[0] == 0x11 and res[1] == index:
                        print(f"  🌟 MOUSE IDENTIFIED AT INDEX {hex(index)}!")
                        return index, path
            dev.close()
        except Exception:
            continue
    
    print("\n[-] Still no response. Try switching the mouse channel button on the bottom.")
    return None, None

if __name__ == "__main__":
    brute_force_indices()
