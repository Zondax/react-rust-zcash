package expo.modules.myrustmodule;

import java.util.Arrays;

public class TinData {
    public long[] path; // Array of 5 u32 values stored as long in Java
    public String address; // Hex-encoded string representing a Script
    public long value; // u64 value representing an Amount

    // Default constructor needed for JNI
    public TinData() {
        this.path = new long[5];
    }

    public TinData(long[] path, String address, long value) {
        this.path = path;
        this.address = address;
        this.value = value;
    }

    // Getters and setters
    public long[] getPath() { return path; }
    public void setPath(long[] path) { this.path = path; }

    public String getAddress() { return address; }
    public void setAddress(String address) { this.address = address; }

    public long getValue() { return value; }
    public void setValue(long value) { this.value = value; }

    // Override equals and hashCode for proper comparison
    @Override
    public boolean equals(Object o) {
        if (this == o) return true;
        if (o == null || getClass() != o.getClass()) return false;
        TinData tinData = (TinData) o;
        return value == tinData.value &&
                Arrays.equals(path, tinData.path) &&
                address.equals(tinData.address);
    }

    @Override
    public int hashCode() {
        int result = Arrays.hashCode(path);
        result = 31 * result + address.hashCode();
        result = 31 * result + (int) (value ^ (value >>> 32));
        return result;
    }
}
