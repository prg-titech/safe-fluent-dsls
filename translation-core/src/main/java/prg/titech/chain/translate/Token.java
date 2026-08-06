package prg.titech.chain.translate;

import jakarta.annotation.Nonnull;

public record Token(
        String image,
        long startByte,
        long endByte,
        Point start,
        Point end
) {
    @Override
    @Nonnull
    public String toString() {
        return image;
    }

    public String toDebugString() {
        return "<\"" + image.replace("\"", "\\\"") + "\", " + startByte + "-" + endByte + ", " + start + "-" + end + ">" ;
    }
}
