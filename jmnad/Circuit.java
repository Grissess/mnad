package jmnad;

import java.lang.ref.Cleaner;

public class Circuit implements AutoCloseable {
    private long rust_data;
    private final Cleaner.Cleanable cleanable;

    static {
        System.loadLibrary("mnad");
    }

    public native long nativeInit();

    public Circuit() {
        rust_data = nativeInit();
        cleanable = MnadCleaner.cleaner.register(this, new Clean(rust_data));
    }

    public static native void dispose(long rust_data);

    public native Bipole add_bipole(int kind, double val);

    public void close() {
        cleanble.clean();
    }

    private static class Clean implements Runnable {
        private long rust_data; 

        public Clean(long rust_data) {
            this.rust_data = rust_data;
        }

        @Override
        public void run() {
            dispose(rust_data);
        }
    }
}