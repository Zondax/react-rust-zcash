package expo.modules.myrustmodule;

public class TransparentInput {
    public String outp;
    public String pk;
    public String address;
    public long value;

    // Default constructor needed for JNI
    public TransparentInput() {
    }

    public TransparentInput(String outp, String pk, String address, long value) {
        this.outp = outp;
        this.pk = pk;
        this.address = address;
        this.value = value;
    }

    // Getters and setters
    public String getOutp() { return outp; }
    public void setOutp(String outp) { this.outp = outp; }

    public String getPk() { return pk; }
    public void setPk(String pk) { this.pk = pk; }

    public String getAddress() { return address; }
    public void setAddress(String address) { this.address = address; }

    public long getValue() { return value; }
    public void setValue(long value) { this.value = value; }
}

