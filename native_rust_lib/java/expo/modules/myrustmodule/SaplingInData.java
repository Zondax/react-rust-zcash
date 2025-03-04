package expo.modules.myrustmodule;

public class SaplingInData {
    public long path; // Single u32 value stored as long
    public String address; // Hex-encoded string representing a PaymentAddress
    public long value; // u64 value representing an Amount

    // Default constructor needed for JNI
    public SaplingInData() {
    }

    public SaplingInData(long path, String address, long value) {
        this.path = path;
        this.address = address;
        this.value = value;
    }

    // Getters and setters
    public long getPath() { return path; }
    public void setPath(long path) { this.path = path; }

    public String getAddress() { return address; }
    public void setAddress(String address) { this.address = address; }

    public long getValue() { return value; }
    public void setValue(long value) { this.value = value; }

    @Override
    public boolean equals(Object o) {
        if (this == o) return true;
        if (o == null || getClass() != o.getClass()) return false;
        SaplingInData that = (SaplingInData) o;
        return path == that.path && value == that.value && address.equals(that.address);
    }

    @Override
    public int hashCode() {
        int result = (int) (path ^ (path >>> 32));
        result = 31 * result + address.hashCode();
        result = 31 * result + (int) (value ^ (value >>> 32));
        return result;
    }
}
