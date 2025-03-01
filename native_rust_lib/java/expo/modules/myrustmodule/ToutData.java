package expo.modules.myrustmodule;

public class ToutData {
    public String address; // Hex-encoded string representing a Script
    public long value; // u64 value representing an Amount

    // Default constructor needed for JNI
    public ToutData() {
    }

    public ToutData(String address, long value) {
        this.address = address;
        this.value = value;
    }

    // Getters and setters
    public String getAddress() { return address; }
    public void setAddress(String address) { this.address = address; }

    public long getValue() { return value; }
    public void setValue(long value) { this.value = value; }

    @Override
    public boolean equals(Object o) {
        if (this == o) return true;
        if (o == null || getClass() != o.getClass()) return false;
        ToutData toutData = (ToutData) o;
        return value == toutData.value && address.equals(toutData.address);
    }

    @Override
    public int hashCode() {
        int result = address.hashCode();
        result = 31 * result + (int) (value ^ (value >>> 32));
        return result;
    }
}
