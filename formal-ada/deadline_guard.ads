package Deadline_Guard
  with SPARK_Mode
is
   subtype Milliseconds is Natural;

   function Within_Deadline
     (Elapsed  : Milliseconds;
      Deadline : Milliseconds) return Boolean
   with
     Pre  => Deadline > 0,
     Post => Within_Deadline'Result = (Elapsed <= Deadline);

   function Confidence_Accepted
     (Confidence     : Float;
      Minimum_Score  : Float) return Boolean
   with
     Pre  => Confidence in 0.0 .. 1.0
             and then Minimum_Score in 0.0 .. 1.0,
     Post => Confidence_Accepted'Result =
             (Confidence >= Minimum_Score);
end Deadline_Guard;
