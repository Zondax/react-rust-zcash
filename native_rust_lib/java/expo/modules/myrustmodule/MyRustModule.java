package expo.modules.myrustmodule;

public class MyRustModule {
    // Load the native library - normally done in your main module class
    static {
        try {
            System.loadLibrary("native_rust_lib");
        } catch (UnsatisfiedLinkError e) {
            System.err.println("Failed to load native library: " + e.getMessage());
        }
    }

    // Native method declarations
    public static native long calculateFee(int nTxin, int nTxout, int nSpend, int nSout);
    public static native long createBuilder(long fee, int networkType);
    public static native int destroyBuilder(long builderId);
    public static native int addTransparentInput(long builderId, TransparentInput input);
    public static native int addTransparentOutput(long builderId, TransparentOutput output);
    public static native byte[] buildTransaction(long builderId, String spendPath, String outputPath, int txVersion);
    public static native int addSignatures(long builderId, Signatures signatures);
    public static native byte[] finalizeTransaction(long builderId);
    public static native String getErrorDescription(int errorCode);
    public static native byte[] getInitTxData(InitData initData);
}
