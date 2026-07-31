package prg.titech.chain.translate;

import java.awt.*;

public record Token(
        String image,
        int startByte,
        int endByte,
        Point start,
        Point end
) { }
