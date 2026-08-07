package prg.titech.chain.token;

public record Range(Position begin, Position end) {

    public Range(Position begin, Position end) {
        if (begin.isBefore(end)) {
            this.begin = begin;
            this.end = end;
        } else if (begin.isAfter(end)) {
            this.begin = end;
            this.end = begin;
        } else {
            throw new IllegalArgumentException("Begin and end of a range cannot be equal! Got begin == end == " + begin);
        }
    }

    public boolean isValid() {
        return begin.isValid() && end.isValid();
    }

}
