package expo.modules.myrustmodule;

import java.util.ArrayList;
import java.util.List;

public class Signatures {
    public List<String> transparentSigs;
    public List<String> saplingSigs;
    
    public Signatures() {
        this.transparentSigs = new ArrayList<>();
        this.saplingSigs = new ArrayList<>();
    }
    
    public List<String> getTransparentSigs() {
        return transparentSigs;
    }
    
    public void setTransparentSigs(List<String> transparentSigs) {
        this.transparentSigs = transparentSigs;
    }
    
    public List<String> getSaplingSigs() {
        return saplingSigs;
    }
    
    public void setSaplingSigs(List<String> saplingSigs) {
        this.saplingSigs = saplingSigs;
    }
    
    public void addTransparentSig(String signature) {
        this.transparentSigs.add(signature);
    }
    
    public void addSaplingSig(String signature) {
        this.saplingSigs.add(signature);
    }
}
