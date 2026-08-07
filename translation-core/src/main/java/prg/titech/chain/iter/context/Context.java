package prg.titech.chain.iter.context;

import jakarta.annotation.Nullable;

import java.util.ArrayDeque;
import java.util.Deque;

public class Context {
    private @Nullable String currentMethod = null;

    public void setCurrentMethod(@Nullable String currentMethod) {
        this.currentMethod = currentMethod;
    }

    private final Deque<Frame> frames = new ArrayDeque<>();

    public Frame getCurrentFrame() {
        return frames.getLast();
    }

    public void descent() {
        frames.add(new Frame(currentMethod));
    }

    public void ascent() {
        frames.removeLast();
    }

    public void setLastPosition() {
        frames.getLast().setLastPosition();
    }
}
