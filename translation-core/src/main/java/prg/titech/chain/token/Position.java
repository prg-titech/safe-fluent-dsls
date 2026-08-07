package prg.titech.chain.token;

import jakarta.annotation.Nonnull;
import org.jspecify.annotations.NonNull;

import java.util.Objects;

public record Position(int line, int column) implements Comparable<Position> {
    private static final Position HOME = new Position(1, 1);

    public boolean isValid() {
        return line >= HOME.line && column >= HOME.column;
    }

    @Override
    public int compareTo(@NonNull Position o) {
        Position toCompare = this.minus(o);
        if (toCompare.line < 0 || (toCompare.line == 0 && toCompare.column < 0)) {
            return -1;
        } else if (toCompare.line == 0 && toCompare.column == 0) {
            return 0;
        } else {
            return 1;
        }
    }

    public boolean isBefore(@Nonnull Position o) {
        return this.compareTo(o) < 0;
    }

    public boolean isAfter(@Nonnull Position o) {
        return this.compareTo(o) > 0;
    }

    private Position negate() {
        return new Position(-line, -column);
    }

    private Position plus(Position other) {
        return new Position(line + other.line, column + other.column);
    }

    private Position minus(Position other) {
        return plus(other.negate());
    }

    @Override
    public boolean equals(Object obj) {
        if (this == obj) {
            return true;
        }
        if (obj instanceof Position(int otherLine, int otherColumn)) {
            return line == otherLine && column == otherColumn;
        } else {
            return false;
        }
    }

    @Override
    public int hashCode() {
        return Objects.hash(line, column);
    }

    @Override
    public @Nonnull String toString() {
        return String.format("line %d, column %d", line, column);
    }
}
