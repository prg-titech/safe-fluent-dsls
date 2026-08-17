package prg.titech.chain.token;

import jakarta.annotation.Nonnull;

import java.util.Objects;

public record Range(Position begin, Position end) {

    public Range(Position begin, Position end) {
        if (begin.isAfter(end)) {
            this.begin = end;
            this.end = begin;
        } else {
            this.begin = begin;
            this.end = end;
        }
    }

    public static Range from(com.github.javaparser.Range range) {
        return new Range(Position.from(range.begin), Position.from(range.end));
    }

    @Override
    public boolean equals(Object obj) {
        if (this == obj) {
            return true;
        }
        if (obj == null) {
            return false;
        }
        if (obj instanceof Range(Position otherBegin, Position otherEnd)) {
            return this.begin.equals(otherBegin) && this.end.equals(otherEnd);
        } else {
            return false;
        }
    }

    public boolean isBefore(Range other) {
        return this.end.isBefore(other.begin());
    }

    @Override
    public int hashCode() {
        return Objects.hash(this.begin, this.end);
    }

    @Override
    public @Nonnull String toString() {
        return this.begin + "-" + this.end;
    }
}
