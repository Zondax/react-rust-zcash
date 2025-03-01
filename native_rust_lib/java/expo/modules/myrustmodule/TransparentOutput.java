package expo.modules.myrustmodule;

public class TransparentOutput {
    public String address;
    public long value;

    // Default constructor needed for JNI
    public TransparentOutput() {
    }

    public TransparentOutput(String address, long value) {
        this.address = address;
        this.value = value;
    }

    // Getters and setters
    public String getAddress() { return address; }
    public void setAddress(String address) { this.address = address; }

    public long getValue() { return value; }
    public void setValue(long value) { this.value = value; }
}

