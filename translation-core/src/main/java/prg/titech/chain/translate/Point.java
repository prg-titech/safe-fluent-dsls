package prg.titech.chain.translate;

import jakarta.annotation.Nonnull;

@SuppressWarnings("unused") public record Point(long x, long y) {

    public Point translate(Point t) {
        return new Point(x + t.x(), y + t.y());
    }

    public Point translate(long dx, long dy) {
        return this.translate(new Point(dx, dy));
    }

    @Override
    @Nonnull
    public String toString() {
        return "(" + x + ", " + y + ")";
    }


}
