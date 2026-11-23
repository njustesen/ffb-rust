import java.io.PrintStream;
import java.util.ArrayList;
import java.util.List;
import java.util.Random;

/** Prints `(0x<bits>, "<Double.toString>"),` for edge-case and random doubles (Rust include! file).
 *  Usage: java DoubleProbe.java <out.in> */
public class DoubleProbe {
    public static void main(String[] args) throws Exception {
        PrintStream out = new PrintStream(args[0], "UTF-8");
        List<Double> v = new ArrayList<>();
        double[] edges = {0.0, -0.0, 1.0, -1.0, 0.1, 0.2, 0.3, 0.1 + 0.2, 1.0 / 3, 2.0 / 3, 1e-3, 1e7, 1e23, 2e23,
            9.999999999999999e22, 1e22, 1e21, 1e-5, 123456.789, 1234567.0, 9999999.0, 9999999.999999998, 10000000.5,
            0.001, 0.0009999999999999998, 0.0010000000000000002, Double.MIN_VALUE, Double.MAX_VALUE, Double.MIN_NORMAL,
            2 * Double.MIN_VALUE, 3 * Double.MIN_VALUE, 1.0499999999999998, 0.7272727272727273, 5e-324, 1e-323,
            2.0E-3, 1.0E-4, 100.0, 1e6, 1e15, 1e16, 1e17, 4.35, 0.5, 0.25, 0.125, 1.5, 2.5, 1e-300, 1e300,
            Math.PI, Math.E, 1.0 / 6.0, 5.0 / 6.0, 1.0 - 1.0 / 36.0, 0.9, 0.95, 0.975, 1.0 + 1e-15};
        for (double d : edges) { v.add(d); v.add(-d); }
        Random r = new Random(42);
        for (int i = 0; i < 1000; i++) {
            long bits = r.nextLong();
            double d = Double.longBitsToDouble(bits);
            if (!Double.isNaN(d) && !Double.isInfinite(d)) v.add(d);
        }
        for (int i = 0; i < 1000; i++) v.add((r.nextDouble() * 4 - 2) * Math.pow(10, r.nextInt(14) - 6));
        for (int i = 0; i < 500; i++) v.add(1.0 + r.nextInt(7) / 6.0 * r.nextInt(5) * 0.05 - r.nextInt(3) * 0.1);
        // tie-prone: exact binary fractions at 15-17 significant digits (bits 0xc307b40843963382)
        for (int i = 0; i < 3000; i++) {
            long base = (long) (r.nextDouble() * 9e16) + 100000000000000L;
            double q = Math.scalb((double) r.nextInt(8), -3);
            v.add((double) base + q);
            v.add(Math.scalb((double) base + q, r.nextInt(200) - 100));
        }
        for (int e = -1074; e <= 1023; e += 7) v.add(Math.scalb(1.0, e));
        for (int e = -323; e <= 308; e++) v.add(Double.parseDouble("1e" + e));
        for (int e = -323; e <= 308; e += 3) for (int m = 1; m <= 9; m++) v.add(Double.parseDouble(m + "e" + e));
        out.println("[");
        for (double d : v) {
            out.println("(0x" + Long.toHexString(Double.doubleToRawLongBits(d)) + ", \"" + Double.toString(d) + "\"),");
        }
        out.println("]");
        out.close();
    }
}
