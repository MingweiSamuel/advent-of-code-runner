﻿int[] n = new int[2000];
int c1 = 0, c2 = 0;

for (int i = 0; i < 2000; i++)
{
    string s = Console.ReadLine();
    for (int j = 0; j < s.Length; j++)
    {
        n[i] = n[i] * 10 + (s[j] - '0');
    }

    if (i >= 3)
    {
        if (n[i - 3] < n[i])
        {
            c2++;
        }

        if (n[i - 1] < n[i])
        {
            c1++;
        }
    }
    else if (i >= 1 && n[i - 1] < n[i])
    {
        c1++;
    }
}

Console.WriteLine(c1);
Console.WriteLine(c2);