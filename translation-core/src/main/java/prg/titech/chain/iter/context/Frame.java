package prg.titech.chain.iter.context;

public class Frame {
    private int position = 0;
    private boolean lastPosition = false;

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
}
