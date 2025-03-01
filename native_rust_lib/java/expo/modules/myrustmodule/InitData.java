package expo.modules.myrustmodule;

import java.util.ArrayList;
import java.util.List;

public class InitData {
    public List<TinData> tIn;
    public List<ToutData> tOut;
    public List<SaplingInData> sSpend;
    public List<SaplingOutData> sOutput;

    // Default constructor needed for JNI
    public InitData() {
        this.tIn = new ArrayList<>();
        this.tOut = new ArrayList<>();
        this.sSpend = new ArrayList<>();
        this.sOutput = new ArrayList<>();
    }

    public InitData(List<TinData> tIn, List<ToutData> tOut, 
                   List<SaplingInData> sSpend, List<SaplingOutData> sOutput) {
        this.tIn = tIn;
        this.tOut = tOut;
        this.sSpend = sSpend;
        this.sOutput = sOutput;
    }

    // Getters and setters
    public List<TinData> getTIn() { return tIn; }
    public void setTIn(List<TinData> tIn) { this.tIn = tIn; }

    public List<ToutData> getTOut() { return tOut; }
    public void setTOut(List<ToutData> tOut) { this.tOut = tOut; }

    public List<SaplingInData> getSSpend() { return sSpend; }
    public void setSSpend(List<SaplingInData> sSpend) { this.sSpend = sSpend; }

    public List<SaplingOutData> getSOutput() { return sOutput; }
    public void setSOutput(List<SaplingOutData> sOutput) { this.sOutput = sOutput; }
}

