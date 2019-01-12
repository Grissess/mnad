package jmnad;

public class Bipole {
    private long rust_data;

    static {
        System.loadLibrary("mnad");
    }

    public Bipole() { }

    public native void dispose();
}