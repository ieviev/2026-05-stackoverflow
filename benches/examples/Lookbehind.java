import java.util.regex.*;

public class Lookbehind {
    static String makeInput(int n) {
        StringBuilder sb = new StringBuilder("From: alice@example.com\n");
        for (int i = 0; i < n; i++) sb.append("some log line with data here\n");
        return sb.toString();
    }

    static int findAll(Pattern p, String input) {
        Matcher m = p.matcher(input);
        int count = 0;
        while (m.find()) count++;
        return count;
    }

    static long bench(Pattern p, String input, int iters) {
        findAll(p, input);
        long t0 = System.nanoTime();
        for (int i = 0; i < iters; i++) findAll(p, input);
        return (System.nanoTime() - t0) / iters;
    }

    public static void main(String[] args) {
        Pattern p = Pattern.compile("(?<=From:.*)alice");
        int[] sizes = {50, 200, 500, 1000, 2000};
        for (int n : sizes) {
            String input = makeInput(n);
            int matches = findAll(p, input);
            int iters = n <= 500 ? 50 : 5;
            long t = bench(p, input, iters);
            System.out.printf("n=%d bytes=%d matches=%d java_ms=%.2f%n", n, input.length(), matches, t/1e6);
        }
    }
}
