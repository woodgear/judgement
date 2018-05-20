[![Build Status](https://travis-ci.com/woodgear/judgement.svg?branch=master)](https://travis-ci.com/woodgear/judgement)

# HOW TO USE
[![asciicast](https://asciinema.org/a/0zNx8DjzVXebxWxA18ZE1tst4.png)](https://asciinema.org/a/0zNx8DjzVXebxWxA18ZE1tst4)
# HOW IT WORK
## term
### paper
a file end with .md which describe how to answer it
### paper dir 
a folder which has a lot of paper.usually is the folder which you download (or git clone)
### examiner
a file  which define what is the correct answer of paper.
one examinee correspond one paper
examinee has sam name and almost same location with paper,the only different is examinee location in the  examinee dir and paper location in the paper dir
### examiner dir 
a folder which has a lot of examiner.usually under paper dir and name as $.PAPER_DIR_NAME.answer
### examinee
a file which you write and represents the answer of paper
examinee must has same name with paper
### how it work
judgement will scan you current dir find paper paper's examinee paper's examiner.and according to examiner to test you examinee  
# HOW ABOUT EXAMINER
now we only support with two type of examiner
# OA(only answer)
a file end with .oa and the first line it what you examinee should output
# QA(question and answer)
a file end with .qa

the odd line is the input of you examinee and the even line is the expect output of you examinee
