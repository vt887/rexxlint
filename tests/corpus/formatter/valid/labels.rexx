/* Valid REXX — labels and PROCEDURE */
main:
    CALL greet 'Alice'
    EXIT

greet: PROCEDURE
    ARG name
    SAY 'Hello,' name
    RETURN
