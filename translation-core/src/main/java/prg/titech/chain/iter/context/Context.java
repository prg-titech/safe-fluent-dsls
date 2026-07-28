package prg.titech.chain.iter.context;

import java.util.ArrayDeque;
import java.util.Deque;

public class Context {
    private final Deque<Frame> frames = new ArrayDeque<>();

    public Frame getCurrentFrame() {
        return frames.getLast();
    }

    public void descent() {
        frames.add(new Frame());
    }

    public void ascent() {
        frames.removeLast();
    }

    public void setLastPosition() {
        frames.getLast().setLastPosition();
    }
}
