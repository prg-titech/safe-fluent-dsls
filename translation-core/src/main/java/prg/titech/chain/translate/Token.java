package prg.titech.chain.translate;

import jakarta.annotation.Nonnull;

import java.awt.*;

public record Token(
        String image,
        int startByte,
        int endByte,
        Point start,
        Point end
) {
    @Override
    @Nonnull
    public String toString() {
        return image;
    }

    public String toDebugString() {
        return "<\"" + image.replace(">", ">>") + "\", " + startByte + "-" + endByte + ">" ;
    }
}
