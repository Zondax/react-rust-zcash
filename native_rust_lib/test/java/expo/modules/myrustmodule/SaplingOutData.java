package expo.modules.myrustmodule;

import java.util.Arrays;

public class SaplingOutData {
    public String address; // Hex-encoded string representing a PaymentAddress
    public long value; // u64 value representing an Amount
    public byte memoType; // Single byte value
    public boolean hasOvk; // Boolean indicating if ovk is present
    public byte[] ovk; // Optional byte array (32 bytes) representing an OutgoingViewingKey

    // Default constructor needed for JNI
    public SaplingOutData() {
    }

    public SaplingOutData(String address, long value, byte memoType, boolean hasOvk, byte[] ovk) {
        this.address = address;
        this.value = value;
        this.memoType = memoType;
        this.hasOvk = hasOvk;
        this.ovk = ovk;
    }

    // Getters and setters
    public String getAddress() { return address; }
    public void setAddress(String address) { this.address = address; }

    public long getValue() { return value; }
    public void setValue(long value) { this.value = value; }

    public byte getMemoType() { return memoType; }
    public void setMemoType(byte memoType) { this.memoType = memoType; }

    public boolean isHasOvk() { return hasOvk; }
    public void setHasOvk(boolean hasOvk) { this.hasOvk = hasOvk; }

    public byte[] getOvk() { return ovk; }
    public void setOvk(byte[] ovk) { this.ovk = ovk; }

    @Override
    public boolean equals(Object o) {
        if (this == o) return true;
        if (o == null || getClass() != o.getClass()) return false;
        SaplingOutData that = (SaplingOutData) o;
        return value == that.value && 
               memoType == that.memoType && 
               hasOvk == that.hasOvk && 
               address.equals(that.address) && 
               Arrays.equals(ovk, that.ovk);
    }

    @Override
    public int hashCode() {
        int result = address.hashCode();
        result = 31 * result + (int) (value ^ (value >>> 32));
        result = 31 * result + (int) memoType;
        result = 31 * result + (hasOvk ? 1 : 0);
        result = 31 * result + Arrays.hashCode(ovk);
        return result;
    }
}

