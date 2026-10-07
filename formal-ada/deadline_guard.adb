package body Deadline_Guard
  with SPARK_Mode
is
   function Within_Deadline
     (Elapsed  : Milliseconds;
      Deadline : Milliseconds) return Boolean
   is
   begin
      return Elapsed <= Deadline;
   end Within_Deadline;

   function Confidence_Accepted
     (Confidence     : Float;
      Minimum_Score  : Float) return Boolean
   is
   begin
      return Confidence >= Minimum_Score;
   end Confidence_Accepted;
end Deadline_Guard;
