package prg.titech.chain.iter.context;

public class Frame {
    private int position = 0;
    private boolean lastPosition = false;
    private final String currentMethod;

    public Frame(String currentMethod) {
        this.currentMethod = currentMethod;
    }

    @SuppressWarnings("unused") public int getPosition() {
        return position;
    }

    public void incrementPosition() {
        position++;
    }

    public void setLastPosition() {
        lastPosition = true;
    }

    public boolean isFirstPosition() {
        return position == 0;
    }

    public boolean isMiddlePosition() {
        return !isFirstPosition() && !isLastPosition();
    }

    public boolean isLastPosition() {
        return lastPosition;
    }

    public String getCurrentMethod() {
        return currentMethod;
    }
}
